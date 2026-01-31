import { TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { ErrorService } from './error.service';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';

describe('ErrorService', () => {
  let service: ErrorService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [ToastrModule.forRoot(), NgbModalModule],
    });
    service = TestBed.inject(ErrorService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
