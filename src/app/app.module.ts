import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { NgbModalModule, NgbTooltipModule, NgbTypeaheadModule } from '@ng-bootstrap/ng-bootstrap';
import { FormsModule } from '@angular/forms';
import { MatMenuModule } from '@angular/material/menu'
import { KeyboardShortcutsModule } from 'ng-keyboard-shortcuts';
import { ToastrModule } from 'ngx-toastr';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { SetupComponent } from './setup/setup.component';
import { LoadingComponent } from './loading/loading.component';
import { SourceNameExistsValidator } from './setup/validators/source-name-exists-validator.directive';
import { NotEmptyValidatorDirective } from './setup/validators/not-empty-validator.directive';
import { ConfirmModalComponent } from './setup/confirm-modal/confirm-modal.component';
import { HomeComponent } from './home/home.component';
import { ChannelTileComponent } from './channel-tile/channel-tile.component';
import { SettingsComponent } from './settings/settings.component';
import { SourceTileComponent } from './settings/source-tile/source-tile.component';
import { ErrorModalComponent } from './error-modal/error-modal.component';
import { EditChannelModalComponent } from './edit-channel-modal/edit-channel-modal.component';
import { EditGroupModalComponent } from './edit-group-modal/edit-group-modal.component';
import { GroupNameExistsValidator } from './edit-group-modal/validators/group-name-exists.directive';
import { DeleteGroupModalComponent } from './delete-group-modal/delete-group-modal.component';
import { ImportModalComponent } from './import-modal/import-modal.component';
import { ConfirmDeleteModalComponent } from './confirm-delete-modal/confirm-delete-modal.component';
import { EpgModalComponent } from './epg-modal/epg-modal.component';
import { EpgModalItemComponent } from './epg-modal/epg-modal-item/epg-modal-item.component';
import { RestreamModalComponent } from './restream-modal/restream-modal.component';
import { SortButtonComponent } from './home/sort-button/sort-button.component';
import { SortItemComponent } from './home/sort-button/sort-item/sort-item.component';
import { DownloadManagerComponent } from './download-manager/download-manager.component';

@NgModule({
  declarations: [
    AppComponent,
    SetupComponent,
    LoadingComponent,
    SourceNameExistsValidator,
    NotEmptyValidatorDirective,
    ConfirmModalComponent,
    HomeComponent,
    ChannelTileComponent,
    SettingsComponent,
    SourceTileComponent,
    ErrorModalComponent,
    EditChannelModalComponent,
    EditGroupModalComponent,
    GroupNameExistsValidator,
    DeleteGroupModalComponent,
    ImportModalComponent,
    ConfirmDeleteModalComponent,
    EpgModalComponent,
    EpgModalItemComponent,
    RestreamModalComponent,
    SortButtonComponent,
    SortItemComponent,
    DownloadManagerComponent,
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
    NgbModalModule,
    NgbTypeaheadModule
  ],
  providers: [
    provideAnimationsAsync()
  ],
  bootstrap: [AppComponent]
})
export class AppModule { }
