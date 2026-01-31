import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NgbActiveModal, NgbTypeaheadModule } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { FormsModule } from '@angular/forms';
import { EditChannelModalComponent } from './edit-channel-modal.component';

describe('EditChannelModalComponent', () => {
  let component: EditChannelModalComponent;
  let fixture: ComponentFixture<EditChannelModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EditChannelModalComponent],
      imports: [ToastrModule.forRoot(), NgbTypeaheadModule, FormsModule],
      providers: [NgbActiveModal],
    }).compileComponents();

    fixture = TestBed.createComponent(EditChannelModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
