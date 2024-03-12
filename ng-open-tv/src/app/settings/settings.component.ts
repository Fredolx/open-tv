import { AfterViewInit, Component, ElementRef, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { NgbActiveModal, NgbModalRef } from '@ng-bootstrap/ng-bootstrap';
import { ToastrService } from 'ngx-toastr';
import { MemoryService } from '../memory.service';
import { Settings } from '../models/settings';
import { Subscription, debounceTime, distinctUntilChanged, fromEvent, map } from 'rxjs';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss']
})
export class SettingsComponent implements AfterViewInit, OnDestroy {
  loading = false;
  electron: any = (window as any).electronAPI;
  subscriptions: Subscription[] = [];
  @ViewChild('mpvParams') mpvParams!: ElementRef;

  constructor(private router: Router, public memory: MemoryService, private toastr: ToastrService) { }

  ngOnInit(): void {
    this.electron.getSettings().then((x: Settings) => {
      if (x)
        this.memory.Settings = x;
    });
  }

  ngAfterViewInit(): void {
    this.subscriptions.push(
      fromEvent(this.mpvParams.nativeElement, 'keyup').pipe(
        map((event: any) => {
          return event.target.value;
        })
        , debounceTime(500)
        , distinctUntilChanged()
      ).subscribe(async () => {
        await this.updateSettings();
      }));
  }

  async refresh() {
    this.loading = true;
    let result = this.memory.Xtream?.url?.trim() ? (await this.memory.GetXtream()) : (await this.memory.DownloadM3U());
    if (result)
      this.toastr.success("Your channels are now up-to-date");
    else
      this.toastr.error("Error while trying to refresh channels, try again or delete channels cache");
    this.loading = false;
  }

  async deleteCache() {
    if (this.loading == true)
      return;
    this.loading = true;
    await this.electron.deleteCache();
    this.memory.Channels = [];
    this.memory.FavChannels = [];
    this.loading = false;
    this.router.navigateByUrl("setup");
  }

  async selectFolder() {
    let result = await this.electron.selectFolder();
    if (result)
      this.memory.Settings.recordingPath = result;
    await this.updateSettings();
  }

  async goBack() {
    await this.updateSettings();
    this.router.navigateByUrl("channels");
  }

  async updateSettings() {
    if (this.memory.Settings.mpvParams)
      this.memory.Settings.mpvParams = this.memory.Settings.mpvParams?.trim();
    await this.electron.updateSettings(this.memory.Settings);
  }

  canRefresh(): boolean {
    return this.memory.Url?.trim() || this.memory.Xtream?.url?.trim() ? true : false;
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(x => x.unsubscribe());
  }
}
