import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { EpgModalItemComponent } from './epg-modal-item.component';

describe('EpgModalItemComponent', () => {
  let component: EpgModalItemComponent;
  let fixture: ComponentFixture<EpgModalItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EpgModalItemComponent],
      imports: [ToastrModule.forRoot(), NgbModalModule, MatProgressBarModule],
    }).compileComponents();

    fixture = TestBed.createComponent(EpgModalItemComponent);
    component = fixture.componentInstance;
    component.epg = { epg_id: 'test_epg', start: 0, stop: 0, title: 'Test EPG' } as any;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
