import { Component, Input } from '@angular/core';

@Component({
  selector: 'app-loading',
  templateUrl: './loading.component.html',
  styleUrl: './loading.component.css'
})
export class LoadingComponent {
  @Input()
  center: boolean = false;
  count = 0;
  texts: string[] = [
    'Consider donating to Open TV',
    'Loading your channels...',
  ];

  currentText: string = '';

  ngOnInit() {
    this.displayRandomText();
    setInterval(() => this.displayRandomText(), 3500);
  }

  displayRandomText() {
    if (this.count == this.texts.length)
      this.count = 0
    this.currentText = this.texts[this.count++];
  }
}
