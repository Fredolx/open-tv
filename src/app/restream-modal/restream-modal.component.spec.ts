import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { ToastrModule } from 'ngx-toastr';
import { FormsModule } from '@angular/forms';
import { RestreamModalComponent } from './restream-modal.component';

describe('RestreamModalComponent', () => {
  let component: RestreamModalComponent;
  let fixture: ComponentFixture<RestreamModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [RestreamModalComponent],
      imports: [ToastrModule.forRoot(), FormsModule],
      providers: [NgbActiveModal],
    }).compileComponents();

    fixture = TestBed.createComponent(RestreamModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
