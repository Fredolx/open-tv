import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule } from '@ng-bootstrap/ng-bootstrap';
import { FormsModule } from '@angular/forms';
import { SourceTileComponent } from './source-tile.component';

describe('SourceTileComponent', () => {
  let component: SourceTileComponent;
  let fixture: ComponentFixture<SourceTileComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [SourceTileComponent],
      imports: [ToastrModule.forRoot(), NgbModalModule, FormsModule],
    }).compileComponents();

    fixture = TestBed.createComponent(SourceTileComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
