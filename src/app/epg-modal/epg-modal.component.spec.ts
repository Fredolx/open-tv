import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EpgModalComponent } from './epg-modal.component';

describe('EpgModalComponent', () => {
  let component: EpgModalComponent;
  let fixture: ComponentFixture<EpgModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EpgModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(EpgModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
