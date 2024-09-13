pub enum Pending<T> {
    Ready(T),
    Pending(oneshot::Receiver<T>),
    Failed,
}

impl<T> Pending<T> {
    pub fn try_load(&mut self) -> Option<&mut T> {
        replace_with::replace_with_or_abort(self, |me| {
            match me {
                Pending::Pending(rcvr) => match rcvr.try_recv() {
                    Ok(val) => Pending::Ready(val),
                    Err(oneshot::TryRecvError::Empty) => Pending::Pending(rcvr),
                    Err(oneshot::TryRecvError::Disconnected) => Pending::Failed,
                }
                other => other,
            }
        });
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
