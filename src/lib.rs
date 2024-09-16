pub enum Pending<T> {
    Ready(T),
    Pending(oneshot::Receiver<T>),
    Failed,
}

pub enum PendingMap<T,U,F> {
    Ready(U),
    Pending(oneshot::Receiver<T>, F),
    Failed,
    Panicked,
}

impl<T> Pending<T> {
    pub fn try_load(&mut self) -> Option<&mut T> {
        if let Pending::Pending(rcvr) = self {
            match rcvr.try_recv() {
                Ok(val) => {
                    *self = Pending::Ready(val);
                },
                Err(oneshot::TryRecvError::Empty) => {},
                Err(oneshot::TryRecvError::Disconnected) => {
                    *self = Pending::Failed;
                },
            };
        }
        match self {
            Pending::Ready(val) => Some(val),
            _ => None,
        }
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Pending::Failed)
    }

    pub fn new() -> (oneshot::Sender<T>, Self) {
        let (sender, receiver) = oneshot::channel();
        (sender, Self::Pending(receiver))
    }
}

impl<T,U,F: FnOnce(T)->U> PendingMap<T,U,F> {
    pub fn try_load(&mut self) -> Option<&mut U> {
        if let PendingMap::Pending(rcvr, _) = self {
            match rcvr.try_recv() {
                Ok(val) => {
                    let PendingMap::Pending(_, func) = std::mem::replace(self, PendingMap::Panicked) else {unreachable!()};
                    *self = PendingMap::Ready(func(val));
                },
                Err(oneshot::TryRecvError::Empty) => {},
                Err(oneshot::TryRecvError::Disconnected) => {*self = PendingMap::Failed},
            }
        }
        match self {
            PendingMap::Ready(val) => Some(val),
            _ => None,
        }
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, PendingMap::Failed)
    }

    pub fn new(func: F) -> (oneshot::Sender<T>, Self) {
        let (sender, receiver) = oneshot::channel();
        (sender, Self::Pending(receiver, func))
    }
}
