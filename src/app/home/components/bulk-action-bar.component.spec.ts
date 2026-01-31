import { ComponentFixture, TestBed } from '@angular/core/testing';
import { BulkActionBarComponent } from './bulk-action-bar.component';
import { NoopAnimationsModule } from '@angular/platform-browser/animations';

describe('BulkActionBarComponent', () => {
  let component: BulkActionBarComponent;
  let fixture: ComponentFixture<BulkActionBarComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [BulkActionBarComponent, NoopAnimationsModule],
    }).compileComponents();

    fixture = TestBed.createComponent(BulkActionBarComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should emit action event', () => {
    spyOn(component.action, 'emit');
    component.action.emit('Favorite');
    expect(component.action.emit).toHaveBeenCalledWith('Favorite');
  });
});
