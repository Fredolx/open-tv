import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ChannelTileComponent } from './channel-tile.component';

describe('ChannelTileComponent', () => {
  let component: ChannelTileComponent;
  let fixture: ComponentFixture<ChannelTileComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ChannelTileComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(ChannelTileComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
