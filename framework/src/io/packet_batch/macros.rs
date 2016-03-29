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


// macro_rules! address_iterator_return { () => { Option<(*mut u8, usize, Option<&mut Any>, usize)> }}
// macro_rules! payload_iterator_return { () => { Option<(*mut u8, *mut u8, usize, Option<&mut Any>, usize)> }}
