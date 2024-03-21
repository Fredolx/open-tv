import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { HomeComponent } from './home/home.component';
import { SettingsComponent } from './settings/settings.component';
import { SetupComponent } from './setup/setup.component';
import { SourcesComponent } from './sources/sources.component';

const routes: Routes = [
  { path: 'setup', component: SetupComponent },
  { path: 'settings', component: SettingsComponent },
  { path: 'channels', component: HomeComponent },
  { path: '', component: SourcesComponent },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
export class AppRoutingModule {}
