/// Implement [`TileMapView<DynamicAxis>`] on the given type. This is implemented on each type
/// individually rather than on the generic type `T: TileMapView<AxisX> + TileMapView<AxisY>` so as
/// to not conflict with the implementation on `&'a T`.
macro_rules! impl_dynamic {
    (
        $( [ $( $g:tt )* ], )? // Generic arguments.
        $( $T:path )* // Type path.
    ) => {
        impl $( < $( $g )* > )? TileMapView<DynamicAxis> for $( $T )* {
            type View<'a>
                = TileMapIter<
                <<Self as TileMapView<AxisX>>::View<'a> as IntoIterator>::IntoIter,
                <<Self as TileMapView<AxisY>>::View<'a> as IntoIterator>::IntoIter,
            >
            where
                Self: 'a;

            fn view(&self, a: DynamicAxis, idx: usize, range: Range<usize>) -> Self::View<'_> {
                let len = range.len();
                let iter = match a.index() {
                    AxisX::VALUE => crate::tilemap::dynamic::DynamicIter::Col(
                        <Self as TileMapView<AxisX>>::view(&self, AxisX, idx, range).into_iter(),
                    ),
                    AxisY::VALUE => crate::tilemap::dynamic::DynamicIter::Row(
                        <Self as TileMapView<AxisY>>::view(&self, AxisY, idx, range).into_iter(),
                    ),
                    _ => unreachable!(),
                };

                DynamicView::new(a, iter, len)
            }
        }
    };
}
