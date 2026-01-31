import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ContentDetailModalComponent } from './content-detail-modal.component';

describe('ContentDetailModalComponent', () => {
  let component: ContentDetailModalComponent;
  let fixture: ComponentFixture<ContentDetailModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ContentDetailModalComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(ContentDetailModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
