import {
  Component,
  Input,
  Output,
  EventEmitter,
  ElementRef,
  ViewChild,
  AfterViewInit,
  OnChanges,
  SimpleChanges,
} from "@angular/core";
import { Channel } from "../models/channel";

@Component({
  selector: "app-content-row",
  templateUrl: "./content-row.component.html",
  styleUrl: "./content-row.component.css",
})
export class ContentRowComponent implements AfterViewInit, OnChanges {
  @Input() title = "";
  @Input() channels: Channel[] = [];
  @Input() loading = false;
  @Input() viewMode: number = 0;
  @Output() seeAll = new EventEmitter<void>();
  @ViewChild("scrollTrack") scrollTrack!: ElementRef<HTMLDivElement>;

  showLeftArrow = false;
  showRightArrow = false;
  skeletonItems = Array(6).fill(0);

  ngAfterViewInit() {
    setTimeout(() => this.checkArrows(), 150);
  }

  ngOnChanges(changes: SimpleChanges) {
    if (changes["channels"] || changes["loading"]) {
      setTimeout(() => this.checkArrows(), 150);
    }
  }

  scrollLeft() {
    if (!this.scrollTrack) return;
    const el = this.scrollTrack.nativeElement;
    el.scrollBy({ left: -400, behavior: "smooth" });
  }

  scrollRight() {
    if (!this.scrollTrack) return;
    const el = this.scrollTrack.nativeElement;
    el.scrollBy({ left: 400, behavior: "smooth" });
  }

  onScroll() {
    this.checkArrows();
  }

  trackByChannelId(index: number, channel: Channel): number {
    return channel.id!;
  }

  checkArrows() {
    if (!this.scrollTrack) return;
    const el = this.scrollTrack.nativeElement;
    this.showLeftArrow = el.scrollLeft > 0;
    this.showRightArrow = el.scrollLeft + el.clientWidth < el.scrollWidth - 1;
  }
}
