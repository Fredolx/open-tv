import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HomeComponent } from './home.component';
import { MemoryService } from '../memory.service';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { ErrorService } from '../error.service';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { By } from '@angular/platform-browser';
import { MatMenuModule } from '@angular/material/menu';
import { FormsModule } from '@angular/forms';
import { MatTooltipModule } from '@angular/material/tooltip';
import { NgbTooltipModule } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { NoopAnimationsModule } from '@angular/platform-browser/animations';
import { TauriService } from '../services/tauri.service';
import { SettingsService } from '../services/settings.service';
import { PlaylistService } from '../services/playlist.service';
import { PlayerService } from '../services/player.service';
import { of } from 'rxjs';

// Mock MemoryService
class MockMemoryService {
  settings = {
    use_single_column: false,
    max_text_lines: 2,
  };
  Sources = new Map();
  XtreamSourceIds = new Set();
  CustomSourceIds = new Set();
  HideChannels = { subscribe: () => ({ unsubscribe: () => {} }) };
  SetFocus = { subscribe: () => ({ unsubscribe: () => {} }) };
  SetNode = { subscribe: () => ({ unsubscribe: () => {} }) };
  Refresh = { subscribe: () => ({ unsubscribe: () => {} }) };
  Sort = { pipe: () => ({ subscribe: () => ({ unsubscribe: () => {} }) }) };
  tryIPC = async () => {};
}

describe('HomeComponent', () => {
  let component: HomeComponent;
  let fixture: ComponentFixture<HomeComponent>;
  let memoryService: MockMemoryService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [HomeComponent],
      imports: [
        MatMenuModule,
        FormsModule,
        MatTooltipModule,
        NgbTooltipModule,
        ToastrModule.forRoot(),
        NoopAnimationsModule,
      ],
      schemas: [CUSTOM_ELEMENTS_SCHEMA],
      providers: [
        { provide: MemoryService, useClass: MockMemoryService },
        { provide: Router, useValue: { navigateByUrl: () => {} } },
        { provide: ToastrService, useValue: { success: () => {}, info: () => {} } },
        { provide: ErrorService, useValue: { handleError: () => {} } },
        {
          provide: NgbModal,
          useValue: { open: () => ({ componentInstance: {}, result: Promise.resolve() }) },
        },
        { provide: TauriService, useValue: { call: () => Promise.resolve([]) } },
        { provide: SettingsService, useValue: { updateSettings: () => Promise.resolve() } },
        {
          provide: PlaylistService,
          useValue: {
            refreshAll: () => Promise.resolve(),
            checkEpgOnStart: () => {},
            bulkUpdate: () => Promise.resolve(),
            hideChannel: () => Promise.resolve(),
            favoriteChannel: () => Promise.resolve(),
            unfavoriteChannel: () => Promise.resolve(),
          },
        },
        {
          provide: PlayerService,
          useValue: { play: () => Promise.resolve(), addLastWatched: () => Promise.resolve() },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(HomeComponent);
    component = fixture.componentInstance;
    memoryService = TestBed.get(MemoryService);

    // Mock channels
    component.channels = [
      { id: 1, name: 'Channel 1', media_type: 1, favorite: false } as any,
      { id: 2, name: 'Channel 2', media_type: 1, favorite: false } as any,
    ];
    component.channelsVisible = true;

    fixture.detectChanges();
  });

  it('should toggle selection mode', () => {
    expect(component.selectionMode).toBeFalse();
    component.toggleSelectionMode();
    expect(component.selectionMode).toBeTrue();
    component.toggleSelectionMode();
    expect(component.selectionMode).toBeFalse();
  });

  it('should select and deselect channels', () => {
    component.toggleChannelSelection(1);
    expect(component.selectedChannels.has(1)).toBeTrue();
    component.toggleChannelSelection(1);
    expect(component.selectedChannels.has(1)).toBeFalse();
  });
});
