use super::*;
use google_sheets4::Sheets;

impl Bot {
    pub fn setup_google_sheets(&mut self) {
        let service_key: yup_oauth2::ServiceAccountKey = serde_json::from_reader(
            std::io::BufReader::new(std::fs::File::open("secrets/service_key.json").unwrap()),
        )
        .unwrap();
        let auth = async_std::task::block_on(
            yup_oauth2::ServiceAccountAuthenticator::builder(service_key).build(),
        )
        .unwrap();

        self.hub = Some(Sheets::new(
            hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
            auth,
        ));
    }

    pub async fn save_to_google_sheets(&self, chat_id: &ChatId) -> google_sheets4::Result<()> {
        use google_sheets4::api::*;
        let active_users = self.get_active_users(chat_id);
        let mut rows = Vec::with_capacity(active_users.map(|users| users.len()).unwrap_or(0) + 1);

        rows.push(values_to_row_data(
            vec!["Username".to_owned()],
            Some(CellFormat {
                text_format: Some(TextFormat {
                    bold: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ));
        if let Some(active_users) = active_users {
            for active_user in active_users {
                rows.push(values_to_row_data(vec![active_user.to_owned()], None));
            }
        }

        let update_values = BatchUpdateSpreadsheetRequest {
            requests: Some(vec![
                Request {
                    update_sheet_properties: Some(UpdateSheetPropertiesRequest {
                        properties: Some(SheetProperties {
                            grid_properties: Some(GridProperties {
                                frozen_row_count: Some(1),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        fields: Some("gridProperties.frozenRowCount".to_owned()),
                    }),
                    ..Default::default()
                },
                Request {
                    repeat_cell: Some(RepeatCellRequest {
                        fields: Some("*".to_owned()),
                        range: Some(GridRange {
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Request {
                    update_cells: Some(UpdateCellsRequest {
                        rows: Some(rows),
                        fields: Some("*".to_owned()),
                        start: Some(GridCoordinate {
                            row_index: Some(0),
                            column_index: Some(0),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let result = self
            .hub
            .as_ref()
            .unwrap()
            .spreadsheets()
            .batch_update(
                update_values,
                &self.config.google_sheet_config.as_ref().unwrap().sheet_id,
            )
            .add_scope(Scope::Spreadsheet)
            .doit()
            .await;

        result.map(|_| ())
    }
}

fn values_to_row_data(
    values: Vec<String>,
    user_entered_format: Option<google_sheets4::api::CellFormat>,
) -> google_sheets4::api::RowData {
    use google_sheets4::api::*;
    let mut cells = Vec::with_capacity(values.len());
    for value in values {
        cells.push(CellData {
            user_entered_value: Some(ExtendedValue {
                string_value: Some(value),
                ..Default::default()
            }),
            user_entered_format: user_entered_format.clone(),
            ..Default::default()
        });
    }
    RowData {
        values: Some(cells),
    }
}
