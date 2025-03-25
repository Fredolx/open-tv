import { ComponentFixture, TestBed } from '@angular/core/testing';

import { WhatsNewModalComponent } from './whats-new-modal.component';

describe('WhatsNewModalComponent', () => {
  let component: WhatsNewModalComponent;
  let fixture: ComponentFixture<WhatsNewModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [WhatsNewModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(WhatsNewModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
