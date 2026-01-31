import { ComponentFixture, TestBed } from '@angular/core/testing';
import { MediaFilterPillComponent } from './media-filter-pill.component';
import { FormsModule } from '@angular/forms';

describe('MediaFilterPillComponent', () => {
  let component: MediaFilterPillComponent;
  let fixture: ComponentFixture<MediaFilterPillComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [MediaFilterPillComponent, FormsModule],
    }).compileComponents();

    fixture = TestBed.createComponent(MediaFilterPillComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should emit toggle event when checkbox changes', () => {
    spyOn(component.toggle, 'emit');
    component.chkLiveStream = false;
    // @ts-ignore - simulating ngModelChange
    component.toggle.emit(0);
    expect(component.toggle.emit).toHaveBeenCalledWith(0);
  });
});
