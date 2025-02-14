use aws_sdk_s3::Client;
use std::fs;
use aws_sdk_s3::primitives::ByteStream;

pub async fn service_s3_check(client: &Client, bucket: &str, s3_path: &str) -> bool {
  let objects = client.list_objects_v2()
      .bucket(bucket)
      .prefix(s3_path)
      .send()
      .await
      .expect("Failed to list objects");

  if let Some(contents) = objects.contents {
      for object in contents {
          if object.key().unwrap() == s3_path {
              return true;
          }
      }
  }

  false
}

pub async fn service_s3_upload(client: &Client, bucket: &str, s3_path: &str, file_path: &str) {
  let file_content = fs::read(file_path).expect("Unable to read file content");
  client.put_object()
      .bucket(bucket)
      .key(s3_path)
      .body(ByteStream::from(file_content))
      .send()
      .await
      .expect("Failed to upload file");
}

pub async fn service_s3_multipart_upload(client: &Client, bucket: &str, key: &str, file_path: &str) {
  use aws_sdk_s3::types::CompletedMultipartUpload;
  use aws_sdk_s3::types::CompletedPart;
  use tokio::fs::File;
  use tokio::io::AsyncReadExt;

  let mut file = File::open(file_path).await.unwrap();
  let file_size = file.metadata().await.unwrap().len();
  let part_size = 5 * 1024 * 1024; // 5MB
  let num_parts = (file_size + part_size - 1) / part_size;

  println!("File size: {}, Part size: {}, Number of parts: {}", file_size, part_size, num_parts);

  let create_multipart_upload = client
      .create_multipart_upload()
      .bucket(bucket)
      .key(key)
      .send()
      .await
      .unwrap();

  let upload_id = create_multipart_upload.upload_id().unwrap().to_string();
  let mut completed_parts = Vec::new();

  for part_number in 1..=num_parts {
      let mut buffer = vec![0; part_size as usize];
      let mut total_bytes_read = 0;

      while total_bytes_read < part_size as usize {
          let bytes_read = file.read(&mut buffer[total_bytes_read..]).await.unwrap();
          if bytes_read == 0 {
              break;
          }
          total_bytes_read += bytes_read;
      }

      buffer.truncate(total_bytes_read);

      println!("Part number: {}, Bytes read: {}", part_number, total_bytes_read);

      // Ensure that all parts except the last one are at least 5MB
      if part_number < num_parts && total_bytes_read < part_size as usize {
          panic!("Part size is smaller than the minimum allowed size");
      }

      let upload_part = client
          .upload_part()
          .bucket(bucket)
          .key(key)
          .upload_id(&upload_id)
          .part_number(part_number as i32)
          .body(buffer.into())
          .send()
          .await
          .unwrap();

      completed_parts.push(
          CompletedPart::builder()
              .part_number(part_number as i32)
              .e_tag(upload_part.e_tag().unwrap().to_string())
              .build(),
      );
  }

  let completed_multipart_upload = CompletedMultipartUpload::builder()
      .set_parts(Some(completed_parts))
      .build();

  client
      .complete_multipart_upload()
      .bucket(bucket)
      .key(key)
      .upload_id(upload_id)
      .multipart_upload(completed_multipart_upload)
      .send()
      .await
      .unwrap();
}