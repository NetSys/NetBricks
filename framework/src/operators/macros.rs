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
            V:Batch + BatchIterator<Header=T> + Act {
        }
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}

macro_rules! act {
    () => {
        #[inline]
        fn act(&mut self) {
            self.parent.act();
        }

        #[inline]
        fn done(&mut self) {
            self.parent.done();
        }

        #[inline]
        fn send_q(&mut self, port: &PacketTx) -> Result<u32> {
            self.parent.send_q(port)
        }

        #[inline]
        fn capacity(&self) -> i32 {
            self.parent.capacity()
        }

        #[inline]
        fn drop_packets(&mut self, idxes: &[usize]) -> Option<usize> {
            self.parent.drop_packets(idxes)
        }

        #[inline]
        fn clear_packets(&mut self) {
            self.parent.clear_packets()
        }

        #[inline]
        fn get_packet_batch(&mut self) -> &mut PacketBatch {
            self.parent.get_packet_batch()
        }

        #[inline]
        fn get_task_dependencies(&self) -> Vec<usize> {
            self.parent.get_task_dependencies()
        }
    }
}
