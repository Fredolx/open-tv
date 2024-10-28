import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DeleteGroupModalComponent } from './delete-group-modal.component';

describe('DeleteGroupModalComponent', () => {
  let component: DeleteGroupModalComponent;
  let fixture: ComponentFixture<DeleteGroupModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [DeleteGroupModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(DeleteGroupModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
