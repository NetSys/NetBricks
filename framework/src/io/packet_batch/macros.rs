macro_rules! batch {
    ($name : ident,  [ $($parts: ident : $pty: ty),* ], [$($defid : ident : $val : expr),*]) => {
        impl<'a, T, V> $name<'a, T, V>
            where T: 'a + EndOffset,
            V:'a + Batch + BatchIterator + Act {
            #[inline]
            pub fn new($( $parts : $pty ),*) -> $name<'a, T, V> {
                $name{ $( $parts: $parts ),*, $($defid : $val),* }
            }
        }

        impl<'a, T, V> Batch for $name<'a, T, V>
            where T: 'a + EndOffset,
            V:'a + Batch + BatchIterator + Act {
            type Parent = V;
            type Header = T;

            fn pop(&mut self) -> &mut V {
                self.parent
            }
        }
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}
