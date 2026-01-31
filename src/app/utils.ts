/**
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

export const isInputFocused = (): boolean => {
  const activeElement = document.activeElement;
  if (!activeElement) return false;
  const inputs = ["input", "select", "button", "textarea"];
  return inputs.indexOf(activeElement.tagName?.toLowerCase()) !== -1;
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
