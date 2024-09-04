use std::time::Duration;

use async_rate_limiter::RateLimiter;
use futures::{
    channel::mpsc::{channel, Sender},
    future::BoxFuture,
    FutureExt, SinkExt, Stream, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::spawn;

use crate::{
    api::{PaginationInfo, PaginationResult},
    block::Block,
    comment::Comment,
    database::Database,
    error::NotionError,
    object::{Object, ObjectList, ObjectType},
    page::Page,
    user::User,
    Api,
};

#[derive(Clone)]
pub struct Fetcher {
    api: Api,
    rate_limiter: RateLimiter,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnyObject {
    Block(Block),
    Page(Page),
    Database(Database),
    User(User),
    Comment(Comment),
}

impl Object for AnyObject {
    fn id(&self) -> &str {
        match self {
            AnyObject::Block(x) => x.id(),
            AnyObject::Page(x) => x.id(),
            AnyObject::Database(x) => x.id(),
            AnyObject::User(x) => x.id(),
            AnyObject::Comment(x) => x.id(),
        }
    }

    fn object_type(&self) -> crate::object::ObjectType {
        match self {
            AnyObject::Block(_) => ObjectType::Block,
            AnyObject::Page(_) => ObjectType::Page,
            AnyObject::Database(_) => ObjectType::Database,
            AnyObject::User(_) => ObjectType::User,
            AnyObject::Comment(_) => ObjectType::Comment,
        }
    }
}

#[derive(Debug, Clone)]
struct Task {
    req_type: ReqType,
}

#[derive(Clone, Debug)]
enum ReqType {
    Block(String),
    Page(String),
    Database(String),

    BlockChildren(PaginationInfo),
    DatabaseQuery(PaginationInfo),
    Comments(PaginationInfo),
}

enum TaskOutput {
    Block(Block),
    Page(Page),
    Database(Database),

    BlockChildren(PaginationResult<Block>),
    QueryDatabase(PaginationResult<AnyObject>),
    Comments(PaginationResult<Comment>),
}

impl<E> TryFrom<Result<PaginationResult<Block>, E>> for TaskOutput {
    type Error = E;
    fn try_from(value: Result<PaginationResult<Block>, E>) -> Result<Self, Self::Error> {
        match value {
            Ok(x) => Ok(TaskOutput::BlockChildren(x)),
            Err(e) => Err(e),
        }
    }
}

impl<E> TryFrom<Result<PaginationResult<AnyObject>, E>> for TaskOutput {
    type Error = E;
    fn try_from(value: Result<PaginationResult<AnyObject>, E>) -> Result<Self, Self::Error> {
        match value {
            Ok(x) => Ok(TaskOutput::QueryDatabase(x)),
            Err(e) => Err(e),
        }
    }
}

impl<E> TryFrom<Result<Block, E>> for TaskOutput {
    type Error = E;
    fn try_from(value: Result<Block, E>) -> Result<Self, Self::Error> {
        match value {
            Ok(x) => Ok(TaskOutput::Block(x)),
            Err(e) => Err(e),
        }
    }
}

impl Fetcher {
    pub fn new(token: &str) -> Fetcher {
        Fetcher {
            api: Api::new(token),
            rate_limiter: {
                let rl = RateLimiter::new(3);
                rl.burst(5);
                rl
            },
        }
    }

    pub async fn fetch(&self, id: &str) -> impl Stream<Item = Result<AnyObject, NotionError>> {
        let (res_tx, res_rx) = channel::<Result<AnyObject, NotionError>>(10);

        // Initial task
        let task = Task {
            req_type: ReqType::Block(id.to_owned()),
        };

        let this = self.clone();
        spawn(async move {
            this.do_task_recurs(task, res_tx).await;
        });

        res_rx
    }

    // Recursive async fn need to be boxed in BoxFuture
    fn do_task_recurs(
        &self,
        task: Task,
        res_tx: Sender<Result<AnyObject, NotionError>>,
    ) -> BoxFuture<'static, ()> {
        let this = self.clone();
        async move {
            let (task_tx, mut task_rx) = channel(10);

            {
                let this = this.clone();
                let res_tx = res_tx.clone();
                spawn(async move {
                    while let Some(task) = task_rx.next().await {
                        this.do_task_recurs(task, res_tx.clone()).await;
                    }
                });
            }

            this.do_task(task, res_tx.clone(), task_tx).await;
        }
        .boxed()
    }

    async fn do_task(
        &self,
        task: Task,
        mut res_tx: Sender<Result<AnyObject, NotionError>>,
        mut task_tx: Sender<Task>,
    ) {
        let res = self.do_request(task).await;
        match res {
            Ok(obj) => {
                match obj {
                    TaskOutput::Page(page) => {
                        // get children
                        let task = Task {
                            req_type: ReqType::BlockChildren(PaginationInfo::new::<
                                ObjectList<Block>,
                            >(
                                page.id()
                            )),
                        };
                        task_tx.send(task).await.unwrap();

                        // get comments
                        let task = Task {
                            req_type: ReqType::Comments(
                                PaginationInfo::new::<ObjectList<Comment>>(page.id()),
                            ),
                        };
                        task_tx.send(task).await.unwrap();

                        res_tx.send(Ok(AnyObject::Page(page))).await.unwrap();
                    }
                    TaskOutput::Database(database) => {
                        let task = Task {
                            req_type: ReqType::DatabaseQuery(PaginationInfo::new::<
                                ObjectList<Block>,
                            >(
                                database.id()
                            )),
                        };
                        task_tx.send(task).await.unwrap();
                        res_tx
                            .send(Ok(AnyObject::Database(database)))
                            .await
                            .unwrap();
                    }
                    TaskOutput::BlockChildren(result) => {
                        for (idx, mut block) in result.result.results.into_iter().enumerate() {
                            block.child_index = result.result.start_index + idx;
                            if let Some(task) = get_task_for_block(&block) {
                                task_tx.send(task).await.unwrap();
                            }
                            res_tx.send(Ok(AnyObject::Block(block))).await.unwrap();
                        }
                        if let Some(pagination) = result.pagination {
                            task_tx
                                .send(Task {
                                    req_type: ReqType::BlockChildren(pagination),
                                })
                                .await
                                .unwrap();
                        }
                    }
                    TaskOutput::QueryDatabase(result) => {
                        for obj in result.result.results {
                            let task = match obj {
                                AnyObject::Database(_) => Task {
                                    req_type: ReqType::DatabaseQuery(PaginationInfo::new::<
                                        ObjectList<AnyObject>,
                                    >(
                                        obj.id()
                                    )),
                                },
                                AnyObject::Page(_) => Task {
                                    req_type: ReqType::BlockChildren(PaginationInfo::new::<
                                        ObjectList<Block>,
                                    >(
                                        obj.id()
                                    )),
                                },
                                AnyObject::Block(_) => unreachable!("shouldn't be a block"),
                                AnyObject::User(_) => unreachable!("shouldn't be a user"),
                                AnyObject::Comment(_) => unreachable!("shouldn't be a comment"),
                            };
                            task_tx.send(task).await.unwrap();
                            res_tx.send(Ok(obj)).await.unwrap();
                        }
                        if let Some(pagination) = result.pagination {
                            task_tx
                                .send(Task {
                                    req_type: ReqType::DatabaseQuery(pagination),
                                })
                                .await
                                .unwrap();
                        }
                    }
                    TaskOutput::Block(block) => {
                        if let Some(task) = get_task_for_block(&block) {
                            task_tx.send(task).await.unwrap();
                        }
                        res_tx.send(Ok(AnyObject::Block(block))).await.unwrap();
                    }
                    TaskOutput::Comments(comments) => {
                        for obj in comments.result.results {
                            res_tx.send(Ok(AnyObject::Comment(obj))).await.unwrap();
                        }
                        if let Some(pagination) = comments.pagination {
                            task_tx
                                .send(Task {
                                    req_type: ReqType::Comments(pagination),
                                })
                                .await
                                .unwrap();
                        }
                    }
                };
            }
            Err(e) => res_tx.send(Err(e)).await.unwrap(),
        }
    }

    async fn do_request(&self, task: Task) -> Result<TaskOutput, NotionError> {
        // Repeatly send request if there is a RetryAfter error, otherwise send
        // the result to the channel.
        loop {
            self.rate_limiter.acquire().await;

            let res = match task.req_type {
                ReqType::Block(ref id) => self
                    .api
                    .get_object::<Block>(id)
                    .await
                    .map(TaskOutput::Block),
                ReqType::Page(ref id) => {
                    self.api.get_object::<Page>(id).await.map(TaskOutput::Page)
                }
                ReqType::Database(ref id) => self
                    .api
                    .get_object::<Database>(id)
                    .await
                    .map(TaskOutput::Database),
                ReqType::BlockChildren(ref pagination) => self
                    .api
                    .list(pagination)
                    .await
                    .map(TaskOutput::BlockChildren),
                ReqType::DatabaseQuery(ref pagination) => self
                    .api
                    .list(pagination)
                    .await
                    .map(TaskOutput::QueryDatabase),
                ReqType::Comments(ref pagination) => {
                    self.api.list(pagination).await.map(TaskOutput::Comments)
                }
            };

            let Err(err) = &res else {
                break res;
            };

            let crate::error::NotionError::RequestFailed(err) = err else {
                break res;
            };

            let crate::api::RequestError::RetryAfter(secs) = err else {
                break res;
            };

            tokio::time::sleep(Duration::from_secs(*secs)).await;
            // should we reset the rate_limiter here?
        }
    }
}

fn get_task_for_block(block: &Block) -> Option<Task> {
    let block_type = &block.block_type;
    let id = block.id().to_owned();
    match block_type {
        crate::block::BlockType::ChildPage => Some(Task {
            req_type: ReqType::Page(id),
        }),
        crate::block::BlockType::ChildDatabase => Some(Task {
            req_type: ReqType::Database(id),
        }),
        _ => {
            if block.has_children {
                Some(Task {
                    req_type: ReqType::BlockChildren(PaginationInfo::new::<ObjectList<Block>>(&id)),
                })
            } else {
                None
            }
        }
    }
}
