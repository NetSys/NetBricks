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

        impl<T, V> Batch for $name<T, V>
            where T: EndOffset,
            V:Batch + BatchIterator + Act {
            type Parent = V;
            type Header = T;

            fn pop(&mut self) -> &mut V {
                &mut self.parent
            }
        }
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}
