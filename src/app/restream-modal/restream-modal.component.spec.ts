import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RestreamModalComponent } from './restream-modal.component';

describe('RestreamModalComponent', () => {
  let component: RestreamModalComponent;
  let fixture: ComponentFixture<RestreamModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [RestreamModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(RestreamModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
