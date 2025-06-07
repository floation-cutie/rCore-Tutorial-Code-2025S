use alloc::vec::Vec;

/// Detector of deadlock
pub struct DeadLockDetector {
    /// mutex detector
    pub mutex: DeadLockDetectorInner,
    /// semaphore detector
    pub semaphore: DeadLockDetectorInner,
}

impl DeadLockDetector {
    /// new deadlock detector
    pub fn new() -> Self {
        Self {
            mutex: DeadLockDetectorInner::new(),
            semaphore: DeadLockDetectorInner::new(),
        }
    }
}

pub struct DeadLockDetectorInner {
    available: Vec<usize>, // mutex/semaphore 数量
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>,
    work: Vec<usize>,
    finish: Vec<bool>,
    max_tid: usize,
}
impl DeadLockDetectorInner {
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            allocation: Vec::new(),
            need: Vec::new(),
            work: Vec::new(),
            finish: Vec::new(),
            max_tid: 0,
        }
    }

    /// create
    pub fn create(&mut self, rid: usize, count: usize) {
        while rid >= self.available.len() {
            self.available.push(0);
            self.allocation.push(Vec::new());
            self.need.push(Vec::new());
        }
        self.available[rid] = count;
    }
    /// minus
    pub fn minus(&mut self, tid: usize, rid: usize) -> isize {
        if self.max_tid < tid {
            self.max_tid = tid;
        }
        // println!("minus tid:{:?} rid:{:?} avail:{:?}", tid, rid, self.available[rid]);
        if self.available[rid] == 0 {
            while tid >= self.need[rid].len() {
                self.need[rid].push(0);
            }
            while tid >= self.finish.len() {
                self.finish.push(false);
            }
            self.need[rid][tid] += 1;
            // 资源不够时进行检测
            return self.detect();
        }
        self.available[rid] -= 1;
        while tid >= self.allocation[rid].len() {
            self.allocation[rid].push(0);
        }
        self.allocation[rid][tid] += 1;
        0
    }

    /// add
    pub fn add(&mut self, tid: usize, rid: usize) {
        self.available[rid] += 1;
        if tid < self.allocation[rid].len() && self.allocation[rid][tid] > 0 {
            self.allocation[rid][tid] -= 1;
        }
        if tid < self.need[rid].len() && self.need[rid][tid] > 0 {
            self.need[rid][tid] -= 1;
        }
    }

    fn try_finish(&mut self, tid: usize) -> bool {
        if self.finish[tid] {
            return true;
        }
        let mut blocked = false;
        for i in 0..self.need.len() {
            if tid < self.need[i].len() && self.need[i][tid] > self.work[i] {
                blocked = true;
                break;
            }
        }
        // 未阻塞就可以finish
        if !blocked {
            // 计算线程allocation可释放的所有资源
            for rid in 0..self.allocation.len() {
                if tid < self.allocation[rid].len() {
                    self.work[rid] += self.allocation[rid][tid];
                }
            }
            self.finish[tid] = true;
            return true;
        }
        false
    }

    fn detect(&mut self) -> isize {
        // 重置2个Vec
        self.finish.fill(false);
        self.finish.resize(self.max_tid + 1, false);
        self.work = self.available.clone();

        let mut false_count = self.finish.len();
        loop {
            for i in 0..self.finish.len() {
                self.try_finish(i);
            }
            let new_count = self.finish.iter().filter(|&&f| !f).count();
            if new_count == false_count {
                break;
            } else {
                false_count = new_count;
            }
        }
        if false_count > 0 {
            // println!("dead dectected");
            return -0xdead;
        }
        0
    }
}
