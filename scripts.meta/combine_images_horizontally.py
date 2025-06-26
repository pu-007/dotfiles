import os
from PIL import Image


def combine_images_horizontally(image_paths, output_folder="combined_images"):
    """
    将给定路径的图片两两水平拼接，并保存到指定文件夹。
    图片按文件名中的数字排序，数字小的在左，数字大的在右。

    Args:
        image_paths (list): 包含所有图片文件路径的列表。
        output_folder (str): 保存拼接后图片的文件夹名称。
    """
    if not image_paths:
        print("没有找到任何图片文件。")
        return

    # 确保输出文件夹存在
    os.makedirs(output_folder, exist_ok=True)

    # 根据文件名中的数字进行排序
    # 提取数字部分进行排序，例如 '1.jpeg' -> 1, '10.jpeg' -> 10
    def get_image_number(filepath):
        filename = os.path.basename(filepath)
        try:
            # 假设文件名格式是 "数字.扩展名"
            return int(filename.split('.')[0])
        except ValueError:
            return float('inf')  # 如果无法解析数字，则放在最后

    sorted_image_paths = sorted(image_paths, key=get_image_number)

    # 遍历图片列表，每两张进行拼接
    for i in range(0, len(sorted_image_paths), 2):
        if i + 1 < len(sorted_image_paths):
            img1_path = sorted_image_paths[i]
            img2_path = sorted_image_paths[i + 1]

            try:
                img1 = Image.open(img1_path)
                img2 = Image.open(img2_path)

                # 确保两张图片的高度相同，如果不同则调整
                if img1.height != img2.height:
                    print(
                        f"警告: 图片 {os.path.basename(img1_path)} 和 {os.path.basename(img2_path)} 高度不同。将调整为相同高度。"
                    )
                    min_height = min(img1.height, img2.height)
                    img1 = img1.resize(
                        (int(img1.width * min_height / img1.height),
                         min_height))
                    img2 = img2.resize(
                        (int(img2.width * min_height / img2.height),
                         min_height))

                # 创建一张新的空白图片，宽度为两张图片宽度之和，高度为两张图片的高度
                combined_width = img1.width + img2.width
                combined_height = img1.height
                combined_image = Image.new('RGB',
                                           (combined_width, combined_height))

                # 将两张图片粘贴到新图片上
                combined_image.paste(img1, (0, 0))
                combined_image.paste(img2, (img1.width, 0))

                # 构造输出文件名，例如 "1_2_combined.jpeg"
                output_filename = f"{get_image_number(img1_path)}_{get_image_number(img2_path)}_combined.jpeg"
                output_path = os.path.join(output_folder, output_filename)

                combined_image.save(output_path)
                print(
                    f"成功拼接并保存: {os.path.basename(img1_path)} + {os.path.basename(img2_path)} -> {output_path}"
                )

            except Exception as e:
                print(
                    f"处理图片时发生错误 {os.path.basename(img1_path)} 和 {os.path.basename(img2_path)}: {e}"
                )
        else:
            print(
                f"注意: 图片 {os.path.basename(sorted_image_paths[i])} 没有配对图片，跳过。")


if __name__ == "__main__":
    current_directory = os.getcwd()
    image_files = []

    # 查找当前目录下所有 .jpeg 或 .jpg 文件
    for filename in os.listdir(current_directory):
        if filename.lower().endswith(('.jpeg', '.jpg')):
            image_files.append(os.path.join(current_directory, filename))

    combine_images_horizontally(image_files)
    print("\n所有图片拼接任务完成。")
    print(
        f"拼接后的图片保存在 '{os.path.join(current_directory, 'combined_images')}' 文件夹中。"
    )
