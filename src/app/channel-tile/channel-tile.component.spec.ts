import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ToastrModule } from 'ngx-toastr';
import { NgbModalModule, NgbTooltipModule } from '@ng-bootstrap/ng-bootstrap';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { ChannelTileComponent } from './channel-tile.component';

describe('ChannelTileComponent', () => {
  let component: ChannelTileComponent;
  let fixture: ComponentFixture<ChannelTileComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ChannelTileComponent],
      imports: [
        ToastrModule.forRoot(),
        NgbModalModule,
        NgbTooltipModule,
        MatMenuModule,
        MatTooltipModule,
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(ChannelTileComponent);
    component = fixture.componentInstance;
    component.channel = { id: 1, name: 'Test Channel', media_type: 1 } as any;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
