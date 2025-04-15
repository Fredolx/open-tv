import { Component } from "@angular/core";
import { DownloadService } from "./download.service";

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent {
  title = "open-tv";

  constructor(private download: DownloadService) {}

  showDownloadManager() {
    return this.download.Downloads.size > 0;
  }
}
