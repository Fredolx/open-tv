export enum SortType {
  alphabeticalAscending,
  alphabeticalDescending,
  provider,
}

export function getSortTypeText(sortType?: SortType): String {
  switch (sortType) {
    case SortType.alphabeticalAscending:
      return "Sort alphabetically asc";
    case SortType.alphabeticalDescending:
      return "Sort alphabetically desc";
    case SortType.provider:
      return "Sort provider";
  }
  return "";
}
