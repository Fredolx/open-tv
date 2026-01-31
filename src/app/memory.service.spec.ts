import { TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { MemoryService } from './memory.service';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';

describe('MemoryService', () => {
  let service: MemoryService;

  beforeEach(() => {
    (window as any).__TAURI_INTERNALS__ = {
      invoke: () => Promise.resolve(false),
      metadata: {
        tauri: '2.0.0',
      },
    };
    TestBed.configureTestingModule({
      imports: [ToastrModule.forRoot(), NgbModalModule],
    });
    service = TestBed.inject(MemoryService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
