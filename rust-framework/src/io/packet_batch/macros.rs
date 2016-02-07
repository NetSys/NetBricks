macro_rules! parse {
    ($htyp:ty, $parent:expr) => {
            ParsedBatch::<$htyp, Self>::new($parent)
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

            // FIXME: Rename this to something reasonable
            #[inline]
            pub fn parse<T2: EndOffset>(&mut self) -> ParsedBatch<T2, Self> {
                parse!(T2, self)
            }

            #[inline]
            pub fn transform(&'a mut self, transformer: &'a Fn(&mut T)) -> TransformBatch<T, Self> {
                TransformBatch::<T, Self>::new(self, transformer)
            }

            #[inline]
            pub fn deparse(&'a mut self) -> &'a mut V {
                if !self.applied {
                    self.act();
                }
                self.parent
            }

            #[inline]
            pub fn apply(&'a mut self, template: &'a T) -> ApplyBatch<T, Self> {
                ApplyBatch::<T, Self>::new(self, template)
            }
        }
    }
}
