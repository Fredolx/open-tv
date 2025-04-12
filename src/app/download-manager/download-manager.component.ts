import { Component } from "@angular/core";

@Component({
  selector: "app-download-manager",
  templateUrl: "./download-manager.component.html",
  styleUrl: "./download-manager.component.css",
})
export class DownloadManagerComponent {
  isMinimized: boolean = false;
  downloads = [
    { name: "File 1.zip", progress: 75 },
    { name: "Video.mp4", progress: 45 },
    { name: "Image.png", progress: 100 },
  ];
  toggleMinimize() {
    this.isMinimized = !this.isMinimized;
  }
}
