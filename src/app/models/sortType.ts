export enum SortType {
  alphabeticalAscending,
  alphabeticalDescending,
  provider,
  ratingDescending,
  ratingAscending,
  dateAscending,
  dateDescending,
}

export const SORT_TYPES = [
  SortType.alphabeticalAscending,
  SortType.alphabeticalDescending,
  SortType.provider,
  SortType.ratingDescending,
  SortType.dateDescending,
];

export function getSortTypeText(sortType?: SortType): String {
  switch (sortType) {
    case SortType.alphabeticalAscending:
      return 'Alphabetically asc';
    case SortType.alphabeticalDescending:
      return 'Alphabetically desc';
    case SortType.provider:
      return 'Provider';
    case SortType.ratingDescending:
      return 'Highest Rated';
    case SortType.ratingAscending:
      return 'Lowest Rated';
    case SortType.dateDescending:
      return 'Newest Added';
    case SortType.dateAscending:
      return 'Oldest Added';
  }
  return '';
}
