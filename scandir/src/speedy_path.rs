use std::path::{Path, PathBuf};

use speedy::{Context, Reader, Writer};

pub(crate) fn path_str<C: Context>(path: &Path) -> Result<&str, C::Error> {
    path.to_str()
        .ok_or_else(|| speedy::Error::custom(format!("Path is not Unicode: {path:?}")).into())
}

pub(crate) fn write_path<C: Context, W: Writer<C> + ?Sized>(
    path: &Path,
    writer: &mut W,
) -> Result<(), C::Error> {
    writer.write_value(&path_str::<C>(path)?)
}

pub(crate) fn read_path<'a, C: Context, R: Reader<'a, C>>(
    reader: &mut R,
) -> Result<PathBuf, C::Error> {
    reader.read_value::<String>().map(PathBuf::from)
}

pub(crate) fn write_paths<C: Context, W: Writer<C> + ?Sized>(
    paths: &[PathBuf],
    writer: &mut W,
) -> Result<(), C::Error> {
    let paths = paths
        .iter()
        .map(|path| path_str::<C>(path))
        .collect::<Result<Vec<_>, _>>()?;
    writer.write_value(&paths)
}

pub(crate) fn read_paths<'a, C: Context, R: Reader<'a, C>>(
    reader: &mut R,
) -> Result<Vec<PathBuf>, C::Error> {
    Ok(reader
        .read_value::<Vec<String>>()?
        .into_iter()
        .map(PathBuf::from)
        .collect())
}

macro_rules! impl_speedy_path_struct {
    ($type:ty, $path:ident, $($field:ident),* $(,)?) => {
        impl<C: speedy::Context> speedy::Writable<C> for $type {
            fn write_to<W: speedy::Writer<C> + ?Sized>(&self, writer: &mut W) -> Result<(), C::Error> {
                crate::speedy_path::write_path(&self.$path, writer)?;
                $(writer.write_value(&self.$field)?;)*
                Ok(())
            }
        }

        impl<'a, C: speedy::Context> speedy::Readable<'a, C> for $type {
            fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
                Ok(Self {
                    $path: crate::speedy_path::read_path(reader)?,
                    $($field: reader.read_value()?,)*
                })
            }
        }
    };
}
pub(crate) use impl_speedy_path_struct;
