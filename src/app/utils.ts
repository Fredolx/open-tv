export const isInputFocused = () => {
  var activeElement = document.activeElement;
  var inputs = ["input", "select", "button", "textarea"];
  return activeElement && inputs.indexOf(activeElement.tagName?.toLowerCase()) !== -1;
};
