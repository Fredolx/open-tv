import { TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';
import { DownloadService } from './download.service';

describe('DownloadService', () => {
  let service: DownloadService;

  beforeEach(() => {
    (window as any).__TAURI_INTERNALS__ = {
      invoke: () => Promise.resolve(false),
      metadata: {
        tauri: '2.0.0',
      },
      postMessage: () => {},
    };
    TestBed.configureTestingModule({
      imports: [ToastrModule.forRoot(), NgbModalModule],
    });
    service = TestBed.inject(DownloadService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
