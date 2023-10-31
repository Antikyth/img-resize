use std::iter::FusedIterator;

pub trait IteratorExtensions: Iterator {
    /// An iterator over every combination of the items in two iterators.
    ///
    /// `mix(other)` returns a new iterator that will, for each element in the first iterator,
    /// iterate over every element in the second, returning a tuple where the first element comes
    /// from the first iterator and the second element comes from the second iterator.
    ///
    /// The elements from the first iterator are cloned for each returned tuple, and the second
    /// iterator itself is cloned for each element in the first iterator.
    ///
    /// The returned iterator moves onto the next element in the first iterator when the second
    /// iterator returns [`None`].
    ///
    /// If the first iterator is fused - that is, after the first iterator returns [`None`] it will
    /// only ever return [`None`] after that - the returned iterator is, too.
    ///
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use img-resize::IteratorExtensions;
    /// #
    /// let a1 = [1, 2, 3];
    /// let a2 = [4, 5, 6];
    ///
    /// let mut iter = a1.iter().mix(a2.iter());
    ///
    /// assert_eq!(iter.next(), Some((&1, &4));
    /// assert_eq!(iter.next(), Some((&1, &5));
    /// assert_eq!(iter.next(), Some((&1, &6));
    ///
    /// assert_eq!(iter.next(), Some((&2, &4));
    /// assert_eq!(iter.next(), Some((&2, &5));
    /// assert_eq!(iter.next(), Some((&2, &6));
    ///
    /// assert_eq!(iter.next(), Some((&3, &4));
    /// assert_eq!(iter.next(), Some((&3, &5));
    /// assert_eq!(iter.next(), Some((&3, &6));
    ///
    /// assert_eq!(iter.next(), None);
    /// ```
    fn mix<Other: IntoIterator>(self, other: Other) -> Mix<Self, Other::IntoIter>
    where
        Self: Sized,
        Self::Item: Clone,
        Other::IntoIter: Clone,
    {
        Mix::new(self, other.into_iter())
    }

    /// Pairs a clone of the given `item` with every element in the iterator.
    fn pair_with<Item: Clone>(self, item: Item) -> PairWith<Item, Self>
    where
        Self: Sized,
    {
        PairWith::new(item, self)
    }
}

impl<I: Iterator> IteratorExtensions for I {}

/// The [`mix`] iterator adapter.
///
/// [`mix`]: IteratorExtensions::mix
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Mix<First: Iterator, Second: Iterator>
where
    First::Item: Clone,
    Second: Clone,
{
    first: First,
    second: Second,

    pair_with_iter: Option<PairWith<First::Item, Second>>,
}

impl<First: Iterator, Second: Iterator> Mix<First, Second>
where
    First::Item: Clone,
    Second: Clone,
{
    fn new(mut first: First, second: Second) -> Self {
        Self {
            second: second.clone(),

            pair_with_iter: first.next().map(|first| PairWith::new(first, second)),

            first,
        }
    }
}

impl<First: Iterator, Second: Iterator> Iterator for Mix<First, Second>
where
    First::Item: Clone,
    Second: Clone,
{
    type Item = (First::Item, Second::Item);

    fn next(&mut self) -> Option<Self::Item> {
        // If `pair_with_iter` is `Some` (which means the current element in `first` is `Some`) and
        // its next tuple is `Some`, return that tuple.
        if let Some(pair_with_iter) = &mut self.pair_with_iter {
            if let Some(next) = pair_with_iter.next() {
                return Some(next);
            }
        }

        // Otherwise, we move onto the next element in `first`...
        self.pair_with_iter = self
            .first
            .next()
            .map(|item| PairWith::new(item, self.second.clone()));

        // ...and then return the first tuple of that, if any.
        self.pair_with_iter.as_mut().and_then(Iterator::next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (first_min, first_max) = self.first.size_hint();
        let (second_min, second_max) = self.second.size_hint();

        // Minimum size is the first iterator's minimum size multiplied by the second iterator's
        // minimum size, with a maximum size of usize::MAX.
        let min = first_min.checked_mul(second_min).unwrap_or(usize::MAX);
        let max = match (first_max, second_max) {
            // If either iterator has a maximum size of 0 then we cannot mix them, even if the
            // other's maximum size is more than usize::MAX.
            (Some(0), _) | (_, Some(0)) => Some(0),

            // If the maximum size of both the first and second iterators is less than usize::MAX,
            // then multiply them.
            (Some(first_max), Some(second_max)) => first_max.checked_mul(second_max),

            // If the maximum size of either iterator is more than usize::MAX, then the result will
            // be usize::MAX.
            (_, _) => None,
        };

        (min, max)
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.first.count() * self.second.count()
    }
}

impl<First: Iterator, Second: Iterator> FusedIterator for Mix<First, Second>
where
    First::Item: Clone,
    Second: Clone,

    First: FusedIterator,
{
}

impl<First: Iterator, Second: Iterator> DoubleEndedIterator for Mix<First, Second>
where
    First::Item: Clone,
    Second: Clone,

    First: DoubleEndedIterator,
    Second: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // If `pair_with_iter` is `Some` (which means the current element in `first` is `Some`) and
        // its next tuple is `Some`, return that tuple.
        if let Some(pair_with_iter) = &mut self.pair_with_iter {
            if let Some(next_back) = pair_with_iter.next_back() {
                return Some(next_back);
            }
        }

        // Otherwise, we move onto the next element in `first`...
        self.pair_with_iter = self
            .first
            .next_back()
            .map(|item| PairWith::new(item, self.second.clone()));

        // ...and then return the last tuple of that, if any.
        self.pair_with_iter
            .as_mut()
            .and_then(DoubleEndedIterator::next_back)
    }
}

/// The [`pair_with`] iterator adapter.
///
/// [`pair_with`]: IteratorExtensions::pair_with
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PairWith<Item: Clone, Iter: Iterator> {
    item: Item,
    iter: Iter,
}

impl<Item: Clone, Iter: Iterator> PairWith<Item, Iter> {
    fn new(item: Item, iter: Iter) -> Self {
        Self { item, iter }
    }
}

impl<Item: Clone, Iter: Iterator> Iterator for PairWith<Item, Iter> {
    type Item = (Item, Iter::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|next| (self.item.clone(), next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }
}

impl<Item: Clone, Iter: Iterator> FusedIterator for PairWith<Item, Iter> where Iter: FusedIterator {}

impl<Item: Clone, Iter: Iterator> ExactSizeIterator for PairWith<Item, Iter>
where
    Iter: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Item: Clone, Iter: Iterator> DoubleEndedIterator for PairWith<Item, Iter>
where
    Iter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|next| (self.item.clone(), next))
    }
}
