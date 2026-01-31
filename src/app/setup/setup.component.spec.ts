import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule, NgbTooltipModule } from '@ng-bootstrap/ng-bootstrap';
import { FormsModule } from '@angular/forms';
import { SetupComponent } from './setup.component';
import { NotEmptyValidatorDirective } from './validators/not-empty-validator.directive';
import { SourceNameExistsValidator } from './validators/source-name-exists-validator.directive';

describe('SetupComponent', () => {
  let component: SetupComponent;
  let fixture: ComponentFixture<SetupComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [SetupComponent, NotEmptyValidatorDirective, SourceNameExistsValidator],
      imports: [ToastrModule.forRoot(), NgbModalModule, NgbTooltipModule, FormsModule],
    }).compileComponents();

    fixture = TestBed.createComponent(SetupComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
