macro_rules! parse {
    ($htype:ident) => {
        #[inline]
        pub fn parse<$htype: EndOffset>(&mut self) -> ParsedBatch<T2, Self> {
            ParsedBatch::<T2, Self>::new(self)
        }
    }
}

macro_rules! transform {
    ($htype:ty) => {
        #[inline]
        pub fn transform(&'a mut self, transformer: &'a Fn(&mut $htype)) -> TransformBatch<$htype, Self> {
            TransformBatch::<$htype, Self>::new(self, transformer)
        }
    }
}

macro_rules! pop {
    ($htype:ty, $ptype:ty) => {
        #[inline]
        pub fn pop(&'a mut self) -> &'a mut $ptype {
            if !self.applied {
                self.act();
            }
            self.parent
        }
    }
}

macro_rules! replace {
    ($htype: ty) => {
        #[inline]
        pub fn replace(&'a mut self, template: &'a $htype) -> ReplaceBatch<$htype, Self> {
            ReplaceBatch::<$htype, Self>::new(self, template)
        }
    }
}

macro_rules! batch {
    ($name : ident,  [ $($parts: ident : $pty: ty),* ], [$($defid : ident : $val : expr),*]) => {
        impl<'a, T, V> $name<'a, T, V>
            where T: 'a + EndOffset,
            V:'a + ProcessPacketBatch + Act {
            #[inline]
            pub fn new($( $parts : $pty ),*) -> $name<'a, T, V> {
                $name{ applied: false, $( $parts: $parts ),*, $($defid : $val),* }
            }

            parse!{T2}

            transform!{T}

            pop!{T, V}

            replace!{T}
        }
    };
    ($name: ident, [ $($parts: ident : $pty: ty),* ]) => {
        batch!{$name, [$($parts:$pty),*], []}
    }
}
