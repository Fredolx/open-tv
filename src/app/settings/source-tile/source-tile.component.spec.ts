import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SourceTileComponent } from './source-tile.component';

describe('SourceTileComponent', () => {
  let component: SourceTileComponent;
  let fixture: ComponentFixture<SourceTileComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [SourceTileComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(SourceTileComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
