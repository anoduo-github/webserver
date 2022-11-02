use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

/// 线程池
pub struct ThreadPool {
    // 线程队列
    workers: Vec<Worker>,

    // 发送管道
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// 新建线程池
    pub fn new(size: usize) -> ThreadPool {
        //断言大小
        assert!(size > 0);

        //创建管道
        let (sender, receiver) = mpsc::channel();

        //保证线程安全
        let receiver = Arc::new(Mutex::new(receiver));

        //分配空间
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// 执行
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

//执行退出
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        //发送中断信息
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            //等待退出
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// 工作单元
struct Worker {
    //id 记录
    id: usize,

    //工作线程
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// 创建工作单元并执行
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
