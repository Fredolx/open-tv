import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NgbActiveModal, NgbTypeaheadModule } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { FormsModule } from '@angular/forms';
import { DeleteGroupModalComponent } from './delete-group-modal.component';

describe('DeleteGroupModalComponent', () => {
  let component: DeleteGroupModalComponent;
  let fixture: ComponentFixture<DeleteGroupModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [DeleteGroupModalComponent],
      imports: [ToastrModule.forRoot(), NgbTypeaheadModule, FormsModule],
      providers: [NgbActiveModal],
    }).compileComponents();

    fixture = TestBed.createComponent(DeleteGroupModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
