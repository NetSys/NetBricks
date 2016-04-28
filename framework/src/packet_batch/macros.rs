macro_rules! batch {
    ($name : ident,  [ $($parts: ident : $pty: ty),* ], [$($defid : ident : $val : expr),*]) => {
        impl<T, V> $name<T, V>
            where T: EndOffset,
            V:Batch + BatchIterator + Act {
            #[inline]
            pub fn new($( $parts : $pty ),*) -> $name<T, V> {
                $name{ $( $parts: $parts ),*, $($defid : $val),* }
            }
        }
        batch_no_new!{$name}
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}

macro_rules! batch_no_new {
    ($name : ident) => {
        impl<T, V> Batch for $name<T, V>
            where T: EndOffset,
            V:Batch + BatchIterator + Act {}

        impl<T, V> HeaderOperations for $name<T, V>
            where T: EndOffset,
            V:Batch + BatchIterator + Act {
            type Header = T;
        }
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}

macro_rules! act {
    () => {
        #[inline]
        fn parent(&mut self) -> &mut Batch{
            &mut self.parent
        }

        #[inline]
        fn parent_immutable(&self) -> &Batch {
            &self.parent
        }
        #[inline]
        fn act(&mut self) {
            self.parent.act();
        }

        #[inline]
        fn done(&mut self) {
            self.parent.done();
        }

        #[inline]
        fn send_q(&mut self, port: &mut PortQueue) -> Result<u32> {
            self.parent.send_q(port)
        }

        #[inline]
        fn capacity(&self) -> i32 {
            self.parent.capacity()
        }

        #[inline]
        fn drop_packets(&mut self, idxes: Vec<usize>) -> Option<usize> {
            self.parent.drop_packets(idxes)
        }

        #[inline]
        fn adjust_payload_size(&mut self, idx: usize, size: isize) -> Option<isize> {
            self.parent.adjust_payload_size(idx, size)
        }

        #[inline]
        fn adjust_headroom(&mut self, idx: usize, size: isize) -> Option<isize> {
            self.parent.adjust_headroom(idx, size)
        }

        #[inline]
        fn distribute_to_queues(&mut self, queues: &[SpscProducer<u8>], groups: &Vec<(usize, *mut u8)>) {
            self.parent.distribute_to_queues(queues, groups)
        }
    }
}
