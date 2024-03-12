import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { NgbModalModule, NgbTooltipModule } from '@ng-bootstrap/ng-bootstrap';
import { HomeComponent } from './home/home.component';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { ToastrModule } from 'ngx-toastr';
import { ChannelTileComponent } from './channel-tile/channel-tile.component';
import { LoadingComponent } from './loading/loading.component';
import { SettingsComponent } from './settings/settings.component';
import { SetupComponent } from './setup/setup.component';
import { MatMenuModule } from '@angular/material/menu'
import { FormsModule } from '@angular/forms';
import { KeyboardShortcutsModule } from 'ng-keyboard-shortcuts';
import { ConfirmModalComponent } from './confirm-modal/confirm-modal.component';
import { SourcesComponent } from './sources/sources.component';

@NgModule({
  declarations: [
    AppComponent,
    HomeComponent,
    ChannelTileComponent,
    LoadingComponent,
    SettingsComponent,
    SetupComponent,
    ConfirmModalComponent,
    SourcesComponent,
  ],
  imports: [
    BrowserModule,
    FormsModule,
    BrowserAnimationsModule,
    AppRoutingModule,
    NgbTooltipModule,
    ToastrModule.forRoot(),
    KeyboardShortcutsModule.forRoot(),
    MatMenuModule,
    NgbModalModule
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
