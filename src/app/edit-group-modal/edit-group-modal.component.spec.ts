import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EditGroupModalComponent } from './edit-group-modal.component';

describe('EditGroupModalComponent', () => {
  let component: EditGroupModalComponent;
  let fixture: ComponentFixture<EditGroupModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EditGroupModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(EditGroupModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
