// Copyright 2022 Jeremy Edwards
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.28.1
// 	protoc        v3.21.12
// source: internal/unique/proto/unique.proto

package proto

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

// FileSummary is a minimal set of information to determine a file's uniqueness.
type FileSummary struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// Name of the file.
	Name string `protobuf:"bytes,1,opt,name=name,proto3" json:"name,omitempty"`
	// Size of the file in bytes; system-dependent for others.
	Size int64 `protobuf:"varint,2,opt,name=size,proto3" json:"size,omitempty"`
	// Hash of the file.
	Hash string `protobuf:"bytes,3,opt,name=hash,proto3" json:"hash,omitempty"`
	// Should delete file if true.
	ShouldDelete bool `protobuf:"varint,4,opt,name=should_delete,json=shouldDelete,proto3" json:"should_delete,omitempty"`
}

func (x *FileSummary) Reset() {
	*x = FileSummary{}
	if protoimpl.UnsafeEnabled {
		mi := &file_internal_unique_proto_unique_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *FileSummary) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*FileSummary) ProtoMessage() {}

func (x *FileSummary) ProtoReflect() protoreflect.Message {
	mi := &file_internal_unique_proto_unique_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use FileSummary.ProtoReflect.Descriptor instead.
func (*FileSummary) Descriptor() ([]byte, []int) {
	return file_internal_unique_proto_unique_proto_rawDescGZIP(), []int{0}
}

func (x *FileSummary) GetName() string {
	if x != nil {
		return x.Name
	}
	return ""
}

func (x *FileSummary) GetSize() int64 {
	if x != nil {
		return x.Size
	}
	return 0
}

func (x *FileSummary) GetHash() string {
	if x != nil {
		return x.Hash
	}
	return ""
}

func (x *FileSummary) GetShouldDelete() bool {
	if x != nil {
		return x.ShouldDelete
	}
	return false
}

// DuplicateFileReport is the report of all non-unique files.
type DuplicateFileReport struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	// Duplicates found.
	Duplicates []*DuplicateFileReport_DuplicateFileSet `protobuf:"bytes,1,rep,name=duplicates,proto3" json:"duplicates,omitempty"`
	// Title of the report.
	Title string `protobuf:"bytes,2,opt,name=title,proto3" json:"title,omitempty"`
	// Duplicate bytes is the size of data that can be deleted.
	DuplicateBytes int64 `protobuf:"varint,3,opt,name=duplicate_bytes,json=duplicateBytes,proto3" json:"duplicate_bytes,omitempty"`
	// Delete enabled indicates that deletion were actually performed.
	DeleteEnabled bool `protobuf:"varint,4,opt,name=delete_enabled,json=deleteEnabled,proto3" json:"delete_enabled,omitempty"`
}

func (x *DuplicateFileReport) Reset() {
	*x = DuplicateFileReport{}
	if protoimpl.UnsafeEnabled {
		mi := &file_internal_unique_proto_unique_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *DuplicateFileReport) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*DuplicateFileReport) ProtoMessage() {}

func (x *DuplicateFileReport) ProtoReflect() protoreflect.Message {
	mi := &file_internal_unique_proto_unique_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use DuplicateFileReport.ProtoReflect.Descriptor instead.
func (*DuplicateFileReport) Descriptor() ([]byte, []int) {
	return file_internal_unique_proto_unique_proto_rawDescGZIP(), []int{1}
}

func (x *DuplicateFileReport) GetDuplicates() []*DuplicateFileReport_DuplicateFileSet {
	if x != nil {
		return x.Duplicates
	}
	return nil
}

func (x *DuplicateFileReport) GetTitle() string {
	if x != nil {
		return x.Title
	}
	return ""
}

func (x *DuplicateFileReport) GetDuplicateBytes() int64 {
	if x != nil {
		return x.DuplicateBytes
	}
	return 0
}

func (x *DuplicateFileReport) GetDeleteEnabled() bool {
	if x != nil {
		return x.DeleteEnabled
	}
	return false
}

type DuplicateFileReport_DuplicateFileSet struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	File []*FileSummary `protobuf:"bytes,1,rep,name=file,proto3" json:"file,omitempty"`
	// Size of the file in bytes; system-dependent for others.
	Size int64 `protobuf:"varint,2,opt,name=size,proto3" json:"size,omitempty"`
	// Hash of the file.
	Hash string `protobuf:"bytes,3,opt,name=hash,proto3" json:"hash,omitempty"`
}

func (x *DuplicateFileReport_DuplicateFileSet) Reset() {
	*x = DuplicateFileReport_DuplicateFileSet{}
	if protoimpl.UnsafeEnabled {
		mi := &file_internal_unique_proto_unique_proto_msgTypes[2]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *DuplicateFileReport_DuplicateFileSet) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*DuplicateFileReport_DuplicateFileSet) ProtoMessage() {}

func (x *DuplicateFileReport_DuplicateFileSet) ProtoReflect() protoreflect.Message {
	mi := &file_internal_unique_proto_unique_proto_msgTypes[2]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use DuplicateFileReport_DuplicateFileSet.ProtoReflect.Descriptor instead.
func (*DuplicateFileReport_DuplicateFileSet) Descriptor() ([]byte, []int) {
	return file_internal_unique_proto_unique_proto_rawDescGZIP(), []int{1, 0}
}

func (x *DuplicateFileReport_DuplicateFileSet) GetFile() []*FileSummary {
	if x != nil {
		return x.File
	}
	return nil
}

func (x *DuplicateFileReport_DuplicateFileSet) GetSize() int64 {
	if x != nil {
		return x.Size
	}
	return 0
}

func (x *DuplicateFileReport_DuplicateFileSet) GetHash() string {
	if x != nil {
		return x.Hash
	}
	return ""
}

var File_internal_unique_proto_unique_proto protoreflect.FileDescriptor

var file_internal_unique_proto_unique_proto_rawDesc = []byte{
	0x0a, 0x22, 0x69, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x2f, 0x75, 0x6e, 0x69, 0x71, 0x75,
	0x65, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x75, 0x6e, 0x69, 0x71, 0x75, 0x65, 0x2e, 0x70,
	0x72, 0x6f, 0x74, 0x6f, 0x12, 0x1f, 0x66, 0x69, 0x6c, 0x65, 0x74, 0x6f, 0x6f, 0x6c, 0x73, 0x2e,
	0x69, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x2e, 0x75, 0x6e, 0x69, 0x71, 0x75, 0x65, 0x2e,
	0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x6e, 0x0a, 0x0b, 0x46, 0x69, 0x6c, 0x65, 0x53, 0x75, 0x6d,
	0x6d, 0x61, 0x72, 0x79, 0x12, 0x12, 0x0a, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x01,
	0x28, 0x09, 0x52, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x12, 0x12, 0x0a, 0x04, 0x73, 0x69, 0x7a, 0x65,
	0x18, 0x02, 0x20, 0x01, 0x28, 0x03, 0x52, 0x04, 0x73, 0x69, 0x7a, 0x65, 0x12, 0x12, 0x0a, 0x04,
	0x68, 0x61, 0x73, 0x68, 0x18, 0x03, 0x20, 0x01, 0x28, 0x09, 0x52, 0x04, 0x68, 0x61, 0x73, 0x68,
	0x12, 0x23, 0x0a, 0x0d, 0x73, 0x68, 0x6f, 0x75, 0x6c, 0x64, 0x5f, 0x64, 0x65, 0x6c, 0x65, 0x74,
	0x65, 0x18, 0x04, 0x20, 0x01, 0x28, 0x08, 0x52, 0x0c, 0x73, 0x68, 0x6f, 0x75, 0x6c, 0x64, 0x44,
	0x65, 0x6c, 0x65, 0x74, 0x65, 0x22, 0xe0, 0x02, 0x0a, 0x13, 0x44, 0x75, 0x70, 0x6c, 0x69, 0x63,
	0x61, 0x74, 0x65, 0x46, 0x69, 0x6c, 0x65, 0x52, 0x65, 0x70, 0x6f, 0x72, 0x74, 0x12, 0x65, 0x0a,
	0x0a, 0x64, 0x75, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x65, 0x73, 0x18, 0x01, 0x20, 0x03, 0x28,
	0x0b, 0x32, 0x45, 0x2e, 0x66, 0x69, 0x6c, 0x65, 0x74, 0x6f, 0x6f, 0x6c, 0x73, 0x2e, 0x69, 0x6e,
	0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x2e, 0x75, 0x6e, 0x69, 0x71, 0x75, 0x65, 0x2e, 0x70, 0x72,
	0x6f, 0x74, 0x6f, 0x2e, 0x44, 0x75, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x65, 0x46, 0x69, 0x6c,
	0x65, 0x52, 0x65, 0x70, 0x6f, 0x72, 0x74, 0x2e, 0x44, 0x75, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74,
	0x65, 0x46, 0x69, 0x6c, 0x65, 0x53, 0x65, 0x74, 0x52, 0x0a, 0x64, 0x75, 0x70, 0x6c, 0x69, 0x63,
	0x61, 0x74, 0x65, 0x73, 0x12, 0x14, 0x0a, 0x05, 0x74, 0x69, 0x74, 0x6c, 0x65, 0x18, 0x02, 0x20,
	0x01, 0x28, 0x09, 0x52, 0x05, 0x74, 0x69, 0x74, 0x6c, 0x65, 0x12, 0x27, 0x0a, 0x0f, 0x64, 0x75,
	0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x65, 0x5f, 0x62, 0x79, 0x74, 0x65, 0x73, 0x18, 0x03, 0x20,
	0x01, 0x28, 0x03, 0x52, 0x0e, 0x64, 0x75, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x65, 0x42, 0x79,
	0x74, 0x65, 0x73, 0x12, 0x25, 0x0a, 0x0e, 0x64, 0x65, 0x6c, 0x65, 0x74, 0x65, 0x5f, 0x65, 0x6e,
	0x61, 0x62, 0x6c, 0x65, 0x64, 0x18, 0x04, 0x20, 0x01, 0x28, 0x08, 0x52, 0x0d, 0x64, 0x65, 0x6c,
	0x65, 0x74, 0x65, 0x45, 0x6e, 0x61, 0x62, 0x6c, 0x65, 0x64, 0x1a, 0x7c, 0x0a, 0x10, 0x44, 0x75,
	0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x65, 0x46, 0x69, 0x6c, 0x65, 0x53, 0x65, 0x74, 0x12, 0x40,
	0x0a, 0x04, 0x66, 0x69, 0x6c, 0x65, 0x18, 0x01, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x2c, 0x2e, 0x66,
	0x69, 0x6c, 0x65, 0x74, 0x6f, 0x6f, 0x6c, 0x73, 0x2e, 0x69, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61,
	0x6c, 0x2e, 0x75, 0x6e, 0x69, 0x71, 0x75, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2e, 0x46,
	0x69, 0x6c, 0x65, 0x53, 0x75, 0x6d, 0x6d, 0x61, 0x72, 0x79, 0x52, 0x04, 0x66, 0x69, 0x6c, 0x65,
	0x12, 0x12, 0x0a, 0x04, 0x73, 0x69, 0x7a, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x03, 0x52, 0x04,
	0x73, 0x69, 0x7a, 0x65, 0x12, 0x12, 0x0a, 0x04, 0x68, 0x61, 0x73, 0x68, 0x18, 0x03, 0x20, 0x01,
	0x28, 0x09, 0x52, 0x04, 0x68, 0x61, 0x73, 0x68, 0x42, 0x35, 0x5a, 0x33, 0x67, 0x69, 0x74, 0x68,
	0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x6a, 0x65, 0x72, 0x65, 0x6d, 0x79, 0x6a, 0x65, 0x2f,
	0x66, 0x69, 0x6c, 0x65, 0x74, 0x6f, 0x6f, 0x6c, 0x73, 0x2f, 0x69, 0x6e, 0x74, 0x65, 0x72, 0x6e,
	0x61, 0x6c, 0x2f, 0x75, 0x6e, 0x69, 0x71, 0x75, 0x65, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x62,
	0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
}

var (
	file_internal_unique_proto_unique_proto_rawDescOnce sync.Once
	file_internal_unique_proto_unique_proto_rawDescData = file_internal_unique_proto_unique_proto_rawDesc
)

func file_internal_unique_proto_unique_proto_rawDescGZIP() []byte {
	file_internal_unique_proto_unique_proto_rawDescOnce.Do(func() {
		file_internal_unique_proto_unique_proto_rawDescData = protoimpl.X.CompressGZIP(file_internal_unique_proto_unique_proto_rawDescData)
	})
	return file_internal_unique_proto_unique_proto_rawDescData
}

var file_internal_unique_proto_unique_proto_msgTypes = make([]protoimpl.MessageInfo, 3)
var file_internal_unique_proto_unique_proto_goTypes = []interface{}{
	(*FileSummary)(nil),                          // 0: filetools.internal.unique.proto.FileSummary
	(*DuplicateFileReport)(nil),                  // 1: filetools.internal.unique.proto.DuplicateFileReport
	(*DuplicateFileReport_DuplicateFileSet)(nil), // 2: filetools.internal.unique.proto.DuplicateFileReport.DuplicateFileSet
}
var file_internal_unique_proto_unique_proto_depIdxs = []int32{
	2, // 0: filetools.internal.unique.proto.DuplicateFileReport.duplicates:type_name -> filetools.internal.unique.proto.DuplicateFileReport.DuplicateFileSet
	0, // 1: filetools.internal.unique.proto.DuplicateFileReport.DuplicateFileSet.file:type_name -> filetools.internal.unique.proto.FileSummary
	2, // [2:2] is the sub-list for method output_type
	2, // [2:2] is the sub-list for method input_type
	2, // [2:2] is the sub-list for extension type_name
	2, // [2:2] is the sub-list for extension extendee
	0, // [0:2] is the sub-list for field type_name
}

func init() { file_internal_unique_proto_unique_proto_init() }
func file_internal_unique_proto_unique_proto_init() {
	if File_internal_unique_proto_unique_proto != nil {
		return
	}
	if !protoimpl.UnsafeEnabled {
		file_internal_unique_proto_unique_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*FileSummary); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_internal_unique_proto_unique_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*DuplicateFileReport); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_internal_unique_proto_unique_proto_msgTypes[2].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*DuplicateFileReport_DuplicateFileSet); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_internal_unique_proto_unique_proto_rawDesc,
			NumEnums:      0,
			NumMessages:   3,
			NumExtensions: 0,
			NumServices:   0,
		},
		GoTypes:           file_internal_unique_proto_unique_proto_goTypes,
		DependencyIndexes: file_internal_unique_proto_unique_proto_depIdxs,
		MessageInfos:      file_internal_unique_proto_unique_proto_msgTypes,
	}.Build()
	File_internal_unique_proto_unique_proto = out.File
	file_internal_unique_proto_unique_proto_rawDesc = nil
	file_internal_unique_proto_unique_proto_goTypes = nil
	file_internal_unique_proto_unique_proto_depIdxs = nil
}
