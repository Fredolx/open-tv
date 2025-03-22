export enum SortType {
  alphabeticalAscending,
  alphabeticalDescending,
  provider,
}

export const SORT_TYPES = [
  SortType.alphabeticalAscending,
  SortType.alphabeticalDescending,
  SortType.provider,
];

export function getSortTypeText(sortType?: SortType): String {
  switch (sortType) {
    case SortType.alphabeticalAscending:
      return "Alphabetically asc";
    case SortType.alphabeticalDescending:
      return "Alphabetically desc";
    case SortType.provider:
      return "Provider";
  }
  return "";
}
