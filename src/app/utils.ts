export const isInputFocused = () => {
  var activeElement = document.activeElement;
  var inputs = ["input", "select", "button", "textarea"];
  return activeElement && inputs.indexOf(activeElement.tagName?.toLowerCase()) !== -1;
};

export const sanitizeFileName = (fileName: string) => {
  return (
    fileName
      .replace(/[\/\\:*?"<>|]/g, "_")
      .replace(/[\x00-\x1F\x7F]/g, "")
      .replace(/^\.+/, "")
      .replace(/\.+$/, "")
      .replace(/^\s+|\s+$/g, "") || "untitled"
  );
};

export const getDateFormatted = (): string => {
  return new Date().toISOString().replace(/T|:/g, "-").split(".")[0];
};

export const getExtension = (url: string): string => {
  let split = url.split(".");
  let last = split[split.length - 1];
  if (split.length == 1 || last.startsWith("php?")) return "mp4";
  else return last;
};
