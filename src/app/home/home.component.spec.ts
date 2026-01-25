import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HomeComponent } from './home.component';
import { MemoryService } from '../memory.service';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { ErrorService } from '../error.service';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { CUSTOM_ELEMENTS_SCHEMA } from '@angular/core';
import { By } from '@angular/platform-browser';

// Mock MemoryService
class MockMemoryService {
  settings = {
    use_single_column: false,
    max_text_lines: 2,
  };
  Sources = new Map();
  XtreamSourceIds = new Set();
  CustomSourceIds = new Set();
  HideChannels = { subscribe: () => {} };
  SetFocus = { subscribe: () => {} };
  SetNode = { subscribe: () => {} };
  Refresh = { subscribe: () => {} };
  Sort = { pipe: () => ({ subscribe: () => {} }) };
  tryIPC = async () => {};
}

describe('HomeComponent', () => {
  let component: HomeComponent;
  let fixture: ComponentFixture<HomeComponent>;
  let memoryService: MockMemoryService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [HomeComponent],
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
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(HomeComponent);
    component = fixture.componentInstance;
    memoryService = TestBed.inject(MemoryService) as unknown as MockMemoryService;

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

  it('should apply single column class when setting is enabled', () => {
    memoryService.settings.use_single_column = true;
    fixture.detectChanges();

    // We check the first channel tile
    const tile = fixture.debugElement.query(By.css('app-channel-tile'));
    const classes = tile.nativeElement.className;
    expect(classes).toContain('col-12');
    expect(classes).not.toContain('col-lg-4');
  });

  it('should apply grid classes when single column setting is disabled', () => {
    memoryService.settings.use_single_column = false;
    fixture.detectChanges();

    const tile = fixture.debugElement.query(By.css('app-channel-tile'));
    const classes = tile.nativeElement.className;
    expect(classes).not.toContain('col-12');
    expect(classes).toContain('col-lg-4');
  });

  it('should select and deselect channels', () => {
    component.toggleChannelSelection(1);
    expect(component.selectedChannels.has(1)).toBeTrue();
    component.toggleChannelSelection(1);
    expect(component.selectedChannels.has(1)).toBeFalse();
  });
});
