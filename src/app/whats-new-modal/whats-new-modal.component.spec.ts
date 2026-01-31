import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { WhatsNewModalComponent } from './whats-new-modal.component';

describe('WhatsNewModalComponent', () => {
  let component: WhatsNewModalComponent;
  let fixture: ComponentFixture<WhatsNewModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [WhatsNewModalComponent],
      imports: [ToastrModule.forRoot()],
      providers: [NgbActiveModal],
    }).compileComponents();

    fixture = TestBed.createComponent(WhatsNewModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
