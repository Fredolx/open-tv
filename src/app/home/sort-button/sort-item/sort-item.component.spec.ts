import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';
import { SortItemComponent } from './sort-item.component';

describe('SortItemComponent', () => {
  let component: SortItemComponent;
  let fixture: ComponentFixture<SortItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [SortItemComponent],
      imports: [ToastrModule.forRoot(), NgbModalModule],
    }).compileComponents();

    fixture = TestBed.createComponent(SortItemComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
