import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HomeHeaderComponent } from './home-header.component';
import { MatMenuModule } from '@angular/material/menu';

describe('HomeHeaderComponent', () => {
  let component: HomeHeaderComponent;
  let fixture: ComponentFixture<HomeHeaderComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HomeHeaderComponent, MatMenuModule],
    }).compileComponents();

    fixture = TestBed.createComponent(HomeHeaderComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should emit toggleSelectionMode event', () => {
    spyOn(component.toggleSelectionMode, 'emit');
    component.toggleSelectionMode.emit();
    expect(component.toggleSelectionMode.emit).toHaveBeenCalled();
  });
});
