import { ComponentFixture, TestBed } from '@angular/core/testing';

import { EpgModalItemComponent } from './epg-modal-item.component';

describe('EpgModalItemComponent', () => {
  let component: EpgModalItemComponent;
  let fixture: ComponentFixture<EpgModalItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [EpgModalItemComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(EpgModalItemComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
