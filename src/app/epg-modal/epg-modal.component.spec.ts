import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { EpgModalComponent } from './epg-modal.component';
import { EpgModalItemComponent } from './epg-modal-item/epg-modal-item.component';

describe('EpgModalComponent', () => {
  let component: EpgModalComponent;
  let fixture: ComponentFixture<EpgModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EpgModalComponent, EpgModalItemComponent],
      imports: [ToastrModule.forRoot()],
      providers: [NgbActiveModal],
    }).compileComponents();

    fixture = TestBed.createComponent(EpgModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
